===description===
Same as the bare `iterable` case, but for the generic `iterable<K, V>` docblock
form: expanding it also manufactures an implicit `Traversable` member, which
must resolve as the global built-in interface rather than being qualified
against the file's namespace.
===config===
suppress=UnusedParam
===file===
<?php
namespace Webmozart\Assert;

/**
 * @param iterable<string> $value
 * @return iterable<string>
 */
function passthrough($value): iterable {
    return $value;
}
===expect===
