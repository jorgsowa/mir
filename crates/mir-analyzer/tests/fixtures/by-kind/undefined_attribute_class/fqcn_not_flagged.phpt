===description===
A fully-qualified attribute name (leading backslash) resolves to the global class and does not fire UndefinedAttributeClass when that class exists.
===file===
<?php
#[\Attribute]
class Route {}

#[\Route]
class HomeController {}
===expect===
