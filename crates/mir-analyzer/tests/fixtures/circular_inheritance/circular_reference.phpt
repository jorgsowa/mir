===description===
Circular reference
===file===
<?php
class A extends A {}
===expect===
CircularReference
===ignore===
TODO
