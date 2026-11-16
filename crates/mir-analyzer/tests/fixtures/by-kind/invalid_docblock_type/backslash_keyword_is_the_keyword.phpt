===description===
A leading backslash on a docblock type keyword means the keyword itself; a backslash-qualified class stays a class.
===file===
<?php
/**
 * @param \int $value
 * @param \non-empty-array<\string> $names
 * @param \DateTime $date
 * @return \string
 */
function describe($value, $names, $date) {
    return $value . implode(',', $names) . $date->format('Y');
}

/** @var \bool|\float|\mixed $flag */
$flag = true;
echo describe(1, ['a'], new DateTime()), $flag;
echo describe('one', ['a'], new DateTime());
//            ^^^^^ InvalidArgument: Argument $value of describe() expects 'int', got '"one"'
echo describe(1, ['a'], 'today');
//                      ^^^^^^^ InvalidArgument: Argument $date of describe() expects 'DateTime', got '"today"'

/** @param \UnknownThing $thing */
function unknown($thing): void {
//       ^^^^^^^ UndefinedDocblockClass: Docblock type 'UnknownThing' does not exist
    \var_dump($thing);
}
