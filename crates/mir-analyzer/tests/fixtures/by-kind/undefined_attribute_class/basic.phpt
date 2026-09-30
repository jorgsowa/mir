===description===
UndefinedAttributeClass fires when an attribute class does not exist.
===file===
<?php
#[Route('/home')]
//^^^^^^^^^^^^^^ UndefinedAttributeClass: Attribute class Route does not exist
class HomeController {}
===expect===
