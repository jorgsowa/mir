===description===
Wrong case class
===file===
<?php
class Foo {}
(new foo());
===expect===
InvalidClass
===ignore===
TODO
