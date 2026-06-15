===description===
UndefinedAttributeClass fires when an attribute class does not exist.
===file===
<?php
#[Route('/home')]
class HomeController {}
===expect===
UndefinedAttributeClass@2:2-2:16: Attribute class Route does not exist
