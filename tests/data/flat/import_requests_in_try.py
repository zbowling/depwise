try:
    from requests import get

    print(get("https://www.google.com").text)
except ImportError:
    print("ImportError")
