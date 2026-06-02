# To install requirements run `pip install -r requirements.txt`

# Run this file at same time every day
# 0 8 * * * python main.py

# POSSIBLE SETUPS
# Generate both excel files
# Send email containing all the email from previous day
# Generate just "količine" file, if saop doesn't have import option for lunch cancellations

import psycopg
import os
import openpyxl
import datetime
import smtplib
from dotenv import load_dotenv

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

date = datetime.date.today()
current_date = date.strftime("%Y%m%d")
files_path = "../../generated_files/"

excel_file_name1 = f"kosilo-odjave-{current_date}.xlsx"
excel_file_name2 = f"količine-{current_date}.xlsx"

students = 300 # Number of registered students for lunch

def database():
    """Fetch data from db where is_send = false and than changes that param to true"""
    try:
        conn = psycopg.connect(database_url)

        cur = conn.cursor()

        query1 = """
            SELECT l.user_id, l.date, u.first_name, u.last_name 
            FROM lunch_optouts l
            JOIN users u ON l.user_id = u.id
            WHERE l.is_send = false
        """
        cur.execute(query1)
        rows = cur.fetchall()

        print(f"Odjave kosila: {cur.rowcount}")

        for row in rows:
            db_data.append((row[1], row[2], row[3]))

        # Set is_send to true
        query2 = """
            UPDATE lunch_optouts
            SET is_send = true
            WHERE is_send = false
        """
        cur.execute(query2)
        conn.commit()
        
        #print(db_data)

    except psycopg.Error as e:
        print(f"Error occurred while establishing connection: {e}")
    
def create_mail_file():
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

    workbook.save(filename=files_path+excel_file_name1)

def send_file():
    """Send file on email via smtp"""
    s = smtplib.SMTP('smtp-relay.brevo.com', 587)
    s.starttls()
    s.login(smtp_username, smtp_key)
    message = "Message"
    s.sendmail(from_email, to_email, message)
    s.quit()

def create_quantities_file():
    """Create excel file for managing specific food quantities"""
    workbook = openpyxl.Workbook()
    sheet = workbook.active
    sheet['A1'] = students - len(db_data)

    workbook.save(filename=files_path+excel_file_name2)


database()
print(f"Creating excel files in {files_path}")
create_mail_file()
send_file()
#create_quantities_file() - for now turned off
print("Created!")
