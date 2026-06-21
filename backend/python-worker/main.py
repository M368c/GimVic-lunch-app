# To install requirements run `pip install -r requirements.txt`

# Run this file at same time every day
# 0 8 * * * python main.py

# Send at 8.05 for next day

import psycopg
import os
import openpyxl
import datetime
import pdfplumber
import smtplib
import requests
import numpy as np
from pathlib import Path
from dotenv import load_dotenv
from bs4 import BeautifulSoup
from io import BytesIO

from email.mime.multipart import MIMEMultipart
from email.mime.text import MIMEText
from email.mime.base import MIMEBase
from email import encoders

# Load .env variables
load_dotenv()
database_url = os.getenv("DATABASE_URL")

# SMTP
smtp_username = os.getenv("SMTP_USERNAME")
smtp_key = os.getenv("SMTP_KEY")

# Email data
from_email = os.getenv("FROM_EMAIL")
#reply_to_email = os.getenv("REPLY_TO_EMAIL")
to_email = os.getenv("TO_EMAIL")

db_data = []

today = datetime.date.today()
current_date = today.strftime("%Y%m%d")
tomorrow = today + datetime.timedelta(days=1)
tomorrow_date = tomorrow.strftime("%Y%m%d")
current_year = today.year

files_path = "../../generated_files/" # Change for production
excel_file_lunch = f"kosilo-odjave-{current_date}.xlsx"

# Website url
url = "https://www.gimvic.org/delovanjesole/pouk/koledar/"

def database():
    """Fetch data from db where is_send = false and than changes that param to true"""
    try:
        conn = psycopg.connect(database_url, host="localhost")

        cur = conn.cursor()

        query1 = """
            SELECT l.user_id, l.date, u.first_name, u.last_name 
            FROM lunch_optouts l
            JOIN users u ON l.user_id = u.id
            WHERE l.is_send = false and l.date = %s
        """
        cur.execute(query1, [tomorrow_date])
        rows = cur.fetchall()

        print(f"Odjave kosila: {cur.rowcount}")

        for row in rows:
            db_data.append((row[1], row[2], row[3]))
        
        if len(db_data) != 0:
            return True
        else: return False
    
    except psycopg.Error as e:
        print(f"Error occurred while establishing connection: {e}")

def mark_as_sent():
    try:
        conn = psycopg.connect(database_url, host="localhost")
        cur = conn.cursor()
        query2 = """
            UPDATE lunch_optouts
            SET is_send = true
            WHERE is_send = false and date = %s
        """
        cur.execute(query2, [tomorrow_date])
        conn.commit()
        print("Data successfully marked as sent")

    except psycopg.Error as e:
        print(f"Error occurred while establishing connection: {e}")
        print("Data not marked as sent!")

def create_lunch_file():
    """Create excel file with data from db"""
    workbook = openpyxl.Workbook()
    sheet = workbook.active

    # Names of columns
    names_columns = ["datum", "ime", "priimek"]
    for index in range(len(names_columns)):
        cell = sheet.cell(row=1, column=index+1)
        cell.value = names_columns[index]

    # Data from db
    for index in range(len(db_data)):
        date_cell = sheet.cell(row=index+2, column=1)
        date_cell.value = str(db_data[index][0])

        first_name_cell = sheet.cell(row=index+2, column=2)
        first_name_cell.value = db_data[index][1]

        last_name_cell = sheet.cell(row=index+2, column=3)
        last_name_cell.value = db_data[index][2]

    workbook.save(filename=files_path+excel_file_lunch)
    print("Created!")
    
def send_file():
    """Send file on email via smtp"""
    msg = MIMEMultipart()
    msg["From"] = from_email
    msg["To"] = to_email
    msg["Subject"] = "GimVič lunch app"
    body = f"Odjave od kosila za dan {tomorrow.strftime("%d-%m-%Y")}"
    msg.attach(MIMEText(body, 'plain'))
    
    lunch_file_path = Path(files_path+excel_file_lunch)

    if lunch_file_path.exists():
        attachment_lunch = open(files_path+excel_file_lunch, "rb")
        part = MIMEBase('application', 'octet-stream')
        part.set_payload((attachment_lunch).read())
        encoders.encode_base64(part)
        part.add_header('Content-Disposition', "attachment; filename= %s" % excel_file_lunch)
        msg.attach(part)

        # Send mail
        s = smtplib.SMTP('smtp-relay.brevo.com', 587)
        s.starttls()
        s.login(smtp_username, smtp_key)
        text = msg.as_string()
        s.sendmail(from_email, to_email, text)
        s.quit()

        print("Mail sent")
    else: print("File doesn't exists!")

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

def add_dates_from_july_august(list):
    july_august_dates = np.arange(f"{current_year+1}-07", f"{current_year+1}-09", dtype='datetime64[D]')
    list.extend(d.tolist() for d in july_august_dates)
    return list

def update_db_with_holidays(dates):
    try:
        conn = psycopg.connect(database_url, host="localhost")
        cur = conn.cursor()
        query = """
            INSERT INTO holidays (date) VALUES (%s)
            ON CONFLICT (date) DO NOTHING;
        """
        with conn.cursor() as cur:
            cur.executemany(query, [(d,) for d in dates])
        conn.commit()
        print("Data successfully inserted in holidays table")

    except psycopg.Error as e:
        print(f"Error occurred while establishing connection: {e}")
        print("Data not inserted in holidays table!")

if database():
    print(f"Creating excel file in {files_path}")
    create_lunch_file()
    send_file()
    mark_as_sent()

if today.month == 9:
    pdf_file = get_pdf_from_website()
    holidays_dates = add_dates_from_july_august(parse_pdf(pdf_file))

    update_db_with_holidays(holidays_dates)
