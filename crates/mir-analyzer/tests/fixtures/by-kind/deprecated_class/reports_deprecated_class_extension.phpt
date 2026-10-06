===description===
reports deprecated class extension
===file===
<?php
/** @deprecated use NewBase instead */
class OldBase {}

class Child extends OldBase {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedClass: Class OldBase is deprecated: use NewBase instead
