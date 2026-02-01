# Example Python code with intentional issues for code review testing

import json
import os

# Issue: Using global variables
user_data = {}
counter = 0


# Issue: Using mutable default argument
def add_item(item, items=[]):
    items.append(item)
    return items


# Issue: No type hints, poor error handling
def read_file(filename):
    file = open(filename, "r")
    content = file.read()
    return content


# Issue: Using bare except
def process_data(data):
    try:
        result = int(data) * 2
        return result
    except:
        return None


# Issue: Not using context managers for file operations
def write_config(filename, config):
    f = open(filename, "w")
    f.write(json.dumps(config))
    f.close()


# Issue: Inefficient string concatenation in loop
def build_message(items):
    message = ""
    for item in items:
        message = message + str(item) + ", "
    return message


# Issue: Using eval (security risk)
def calculate(expression):
    return eval(expression)


# Issue: Not following PEP 8 naming conventions
def CalculateTotal(ItemList):
    Total = 0
    for Item in ItemList:
        Total += Item
    return Total


# Issue: Hardcoded credentials
def connect_to_database():
    db_password = "admin123"
    db_user = "root"
    connection_string = f"mysql://{db_user}:{db_password}@localhost/mydb"
    return connection_string


# Issue: Not using list comprehension where appropriate
def filter_even_numbers(numbers):
    result = []
    for num in numbers:
        if num % 2 == 0:
            result.append(num)
    return result


# Issue: Modifying list while iterating
def remove_negatives(numbers):
    for num in numbers:
        if num < 0:
            numbers.remove(num)
    return numbers


# Issue: Using 'is' for value comparison instead of '=='
def check_value(value):
    if value is 10:
        return True
    return False


# Issue: No input validation
def divide_numbers(a, b):
    return a / b


# Issue: Redundant code, could use 'any()'
def has_positive_number(numbers):
    for num in numbers:
        if num > 0:
            return True
    return False


# Issue: Not using 'with' statement for resources
def copy_file(source, destination):
    src = open(source, "rb")
    dst = open(destination, "wb")
    dst.write(src.read())
    src.close()
    dst.close()


# Issue: Poor exception handling, too broad
def load_json(filename):
    try:
        with open(filename) as f:
            return json.load(f)
    except Exception as e:
        print(f"Error: {e}")
        return {}


# Issue: Using star imports
from datetime import *


# Issue: Function doing too many things (violates SRP)
class UserManager:
    def process_user(self, user_id):
        # Fetch user
        user = self.get_user(user_id)

        # Validate user
        if user["age"] < 18:
            raise ValueError("Too young")

        # Send email
        self.send_email(user["email"], "Welcome!")

        # Log to database
        self.log_action(user_id, "processed")

        # Update cache
        self.update_cache(user_id, user)

        return user

    def get_user(self, user_id):
        return {"id": user_id, "age": 25, "email": "user@example.com"}

    def send_email(self, email, message):
        pass

    def log_action(self, user_id, action):
        pass

    def update_cache(self, user_id, user):
        pass


# Issue: Using 'assert' for data validation
def set_age(age):
    assert age > 0, "Age must be positive"
    assert age < 150, "Age too high"
    return age


# Issue: Not using pathlib for file paths
def get_config_path():
    return os.getcwd() + "/config/" + "settings.json"


# Issue: Nested loops with high complexity
def find_duplicates(list1, list2):
    duplicates = []
    for item1 in list1:
        for item2 in list2:
            if item1 == item2:
                duplicates.append(item1)
    return duplicates


# Issue: Magic numbers without explanation
def calculate_price(quantity):
    if quantity > 100:
        return quantity * 9.99 * 0.8
    elif quantity > 50:
        return quantity * 9.99 * 0.9
    else:
        return quantity * 9.99


# Issue: Using deprecated methods
def get_current_time():
    import time

    return time.clock()


if __name__ == "__main__":
    # Issue: No proper entry point logic
    print("Running script...")
    data = process_data("123")
    print(data)
