# To install requirements run `pip install -r requirements.txt`

# Run this file at same time every day
# 0 8 * * * python main.py

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

from holidays import get_pdf_from_website
from holidays import parse_pdf

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

files_path = "../../generated_files/" # Change for production
excel_file_lunch = f"kosilo-odjave-{current_date}.xlsx"

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
    holidays_dates = parse_pdf(pdf_file)

    update_db_with_holidays(holidays_dates)
