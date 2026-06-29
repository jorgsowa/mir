===description===
Instantiating a concrete subclass of an abstract base does not fire AbstractInstantiation.
===file===
<?php
abstract class Base {}
class Concrete extends Base {}
new Concrete();
===expect===
