===description===
Wrong case interface name in implements is reported.
===file===
<?php
interface Countable2 {}
class MyList implements countable2 {}
===expect===
WrongCaseClass@3:0-3:37: Class name 'countable2' has incorrect casing; use 'Countable2'
