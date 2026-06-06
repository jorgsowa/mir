===description===
Wrong case interface name in interface extends is reported.
===file===
<?php
interface Countable2 {}
interface MyCollection extends countable2 {}
===expect===
WrongCaseClass@3:0-3:44: Class name 'countable2' has incorrect casing; use 'Countable2'
