===description===
A class-level `@property`/`@method` docblock type naming a same-namespace
class (no leading `\`, no `use` import) resolves against the file's own
namespace, the same as a native type hint or a real member's `@var`/
`@return` — previously stored unqualified, so it never matched the real
class's FQCN and every check against it (existence, reference recording)
silently failed, including making `Money` look unused since its only
reference was recorded under the wrong (unqualified) key.
===config===
suppress=
===file===
<?php
namespace App;

final class Money {}

/**
 * @property Money $balance
 * @method Money getBalance()
 */
class Wallet {}
===expect===
