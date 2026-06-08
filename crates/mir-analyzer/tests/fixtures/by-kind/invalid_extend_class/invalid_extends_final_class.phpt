===description===
Invalid extends final class
===file===
<?php

final class A {}

class B extends A {}

===expect===
InvalidExtendClass@5:0-5:20: Class B cannot extend final class A
