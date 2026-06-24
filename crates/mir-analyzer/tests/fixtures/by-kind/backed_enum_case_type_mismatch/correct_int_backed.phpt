===description===
P6(a): An int-backed enum where all cases have int values must produce no errors.
Negative int values are valid for int-backed enums.
===file===
<?php
enum HttpStatus: int {
    case Ok = 200;
    case NotFound = 404;
    case Error = 500;
}
===expect===
