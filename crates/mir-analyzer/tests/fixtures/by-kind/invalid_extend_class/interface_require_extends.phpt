===description===
`@phpstan-require-extends` on an interface requires every implementing class to extend the named class. A class that does not is `InvalidExtendClass`; one that extends it, directly or through a parent, is accepted.
===file===
<?php
class RequiredParent
{
}

class Intermediate extends RequiredParent
{
}

/**
 * @phpstan-require-extends RequiredParent
 */
interface NeedsParent
{
}

interface ExtendsNeedsParent extends NeedsParent
{
}

final class BadChild implements NeedsParent
//    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidExtendClass: Class BadChild implements NeedsParent, which requires extending RequiredParent
{
}

final class GoodChild extends RequiredParent implements NeedsParent
{
}

final class Grandchild extends Intermediate implements ExtendsNeedsParent
{
}

final class IndirectBad implements ExtendsNeedsParent
//    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidExtendClass: Class IndirectBad implements NeedsParent, which requires extending RequiredParent
{
}
