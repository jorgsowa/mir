===description===
invalidTypeAlias
===file===
<?php
namespace Barrr;

/**
 * @type CoolType = A|B>
 */

class A {}
===expect===
InvalidDocblock
===ignore===
TODO
