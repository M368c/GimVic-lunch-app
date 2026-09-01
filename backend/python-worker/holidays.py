import datetime
import pdfplumber
import requests
import numpy as np
from bs4 import BeautifulSoup
from io import BytesIO

today = datetime.date.today()
current_year = today.year

# Website url
url = "https://www.gimvic.org/delovanjesole/pouk/koledar/"

def get_pdf_from_website():
    """Get pdf file from website"""
    r = requests.get(url)
    soup = BeautifulSoup(r.text, "html.parser")
    pdf_hrefs = []

    for a in soup.find_all("a", href=True):
        if "mrezni" in a["href"] and a["href"].endswith(".pdf"):
            pdf_hrefs.append(a["href"])

    pdf_url = url+pdf_hrefs[0]
    pdf_response = requests.get(pdf_url)
    pdf_response.raise_for_status()
    return BytesIO(pdf_response.content)

def parse_pdf(pdf_path):
    """Parse holidays from given pdf"""
    months_by_number = {
        "September": [9, current_year],
        "Oktober": [10, current_year],
        "November": [11, current_year],
        "December": [12, current_year],
        "Januar": [1, current_year+1],
        "Februar": [2, current_year+1],
        "Marec": [3, current_year+1],
        "April": [4, current_year+1],
        "Maj": [5, current_year+1],
        "Junij": [6, current_year+1]
    }
    months_list = {}
    holidays = []

    # Colors
    BLUE = (0.871, 0.918, 0.965)
    YELLOW = (1.0, 0.949, 0.8)
    
    pdf = pdfplumber.open(pdf_path)

    page = pdf.pages[0]
    all_rects = page.rects

    for item in all_rects:
        if tuple(item["non_stroking_color"]) == YELLOW:
            rects = check_for_multiple_rects(item, page)
            for i in rects:
                holidays.append([i[0].split(".")[0], i[1]])

        elif tuple(item["non_stroking_color"]) == BLUE:
            rects = check_for_multiple_rects(item, page)
            for i in rects:
                months_list[i[0]] = i[1]

    pdf.close()

    holidays_list = []
    for [day, x0] in holidays:
        month_x0 = min(months_list.values(), key=lambda x:abs(x-x0))
        month = list(filter(lambda key: months_list[key] == month_x0, months_list))
        month_number = months_by_number[month[0]][0]
        year = months_by_number[month[0]][1]
        d = (f"{year}-{month_number}-{day}")
        holidays_list.append(datetime.datetime.strptime(d, "%Y-%m-%d").date())

    # Add july and august days
    july_august_dates = np.arange(f"{current_year+1}-07", f"{current_year+1}-09", dtype='datetime64[D]')
    holidays_list.extend(d.tolist() for d in july_august_dates)
    
    return holidays_list

def check_for_multiple_rects(item, page):
    vlines = [l for l in page.lines if abs(l["x0"] - l["x1"]) < 0.5]
    top, bottom = item["top"], item["bottom"]
    x0, x1 = item["x0"], item["x1"]
    margin = 2.0

    inner_xs = sorted(set(
        round(l["x0"], 1) for l in vlines
        if x0 + margin < l["x0"] < x1 - margin
        and l["top"] <= bottom and l["bottom"] >= top
    ))

    if not inner_xs:
        text = extract_text_from_rect(item, page)
        pieces = [[text, x0]]
    else:
        edges = [x0] + inner_xs + [x1]
        pieces = []
        for ex0, ex1 in zip(edges, edges[1:]):
            sub_rect = {"x0": ex0, "x1": ex1, "top": top, "bottom": bottom}
            text = extract_text_from_rect(sub_rect, page)
            pieces.append([text, ex0])

    return pieces

def extract_text_from_rect(item, page):
    """Extract text from pdf rect"""
    rect = (item["x0"], item["top"], item["x1"], item["bottom"])
    crop_area = page.crop(rect)
    crop_text = crop_area.extract_text()
    return crop_text
