===description===
UndefinedAttributeClass fires when an attribute class does not exist.
===file===
<?php
#[Route('/home')]
class HomeController {}
===expect===
UndefinedAttributeClass@2:3-2:17: Attribute class Route does not exist
