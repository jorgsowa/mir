===description===
Deprecated interface
===file===
<?php
/** @deprecated */
interface Container {}

class A implements Container {}
===expect===
DeprecatedInterface@5:0-5:31: Interface Container is deprecated
