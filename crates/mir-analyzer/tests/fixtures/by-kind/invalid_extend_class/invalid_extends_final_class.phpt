===description===
Invalid extends final class
===file===
<?php

final class A {}

class B extends A {}'

===expect===
InvalidExtendClass@5:0-5:21: Class B cannot extend final class A
ParseError@5:21-5:22: Parse error: unterminated string literal
ParseError@5:22-5:22: Parse error: expected ';' after expression
