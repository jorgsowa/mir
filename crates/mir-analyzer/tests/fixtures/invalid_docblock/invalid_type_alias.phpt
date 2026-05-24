===description===
invalidTypeAlias
===file===
<?php
namespace Barrr;

/**
 * @psalm-type CoolType = A|B>
 */

class A {}
===expect===
InvalidDocblock
===ignore===
TODO
