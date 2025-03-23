import hello
import child.ok as ok
from child.ok import SomethingElse

print(hello.greet())


def main():
    print(hello.greet())
    ok.do_something()
    SomethingElse().do_something_else()


if __name__ == "__main__":
    main()
