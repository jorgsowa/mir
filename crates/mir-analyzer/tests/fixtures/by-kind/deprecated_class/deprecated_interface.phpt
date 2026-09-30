===description===
Deprecated interface
===file===
<?php
/** @deprecated */
interface Container {}

class A implements Container {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedInterface: Interface Container is deprecated
===expect===
