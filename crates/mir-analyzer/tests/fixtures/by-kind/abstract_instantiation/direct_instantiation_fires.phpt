===description===
AbstractInstantiation fires when directly instantiating an abstract class.
===file===
<?php
abstract class Repo {}
new Repo();
//  ^^^^ AbstractInstantiation: Cannot instantiate abstract class Repo
