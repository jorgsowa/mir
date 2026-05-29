===description===
reports deprecated class extension
===file===
<?php
/** @deprecated use NewBase instead */
class OldBase {}

class Child extends OldBase {}
===expect===
DeprecatedClass@5:0-5:30: Class OldBase is deprecated: use NewBase instead
