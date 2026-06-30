===description===
When multiple attributes are on the same element, only the undefined one fires UndefinedAttributeClass; the defined attribute class is not flagged.
===file===
<?php
#[\Attribute]
class Route {}

#[Route]
#[Cache]
class HomeController {}
===expect===
UndefinedAttributeClass@6:2-6:7: Attribute class Cache does not exist
