===description===
reports deprecated class extension
===file===
<?php
/** @deprecated use NewBase instead */
class OldBase {}

class Child extends OldBase {}
===expect===
DeprecatedClass: Class OldBase is deprecated: use NewBase instead
===ignore===
TODO
