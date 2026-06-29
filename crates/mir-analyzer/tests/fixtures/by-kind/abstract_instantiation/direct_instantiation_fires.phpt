===description===
AbstractInstantiation fires when directly instantiating an abstract class.
===file===
<?php
abstract class Repo {}
new Repo();
===expect===
AbstractInstantiation@3:4-3:8: Cannot instantiate abstract class Repo
