===description===
Interface instantiation
===file===
<?php
interface myInterface{}
new myInterface();
//  ^^^^^^^^^^^ InterfaceInstantiation: Cannot instantiate interface myInterface
===expect===
