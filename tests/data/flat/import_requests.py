import requests


def main():
    print(requests.get("https://www.google.com").text)


if __name__ == "__main__":
    main()
