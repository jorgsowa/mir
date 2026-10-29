===description===
A function or method that calls `func_get_args()` / `func_num_args()` keeps its
`@param` types in the body and in docblock checks;
the variadic the collector adds for the extra arguments isn't a parameter mismatch.
===file===
<?php
/**
 * @param string|null $s
 */
function wrap($s = null): string
{
    if (func_num_args() === 0) {
        return '';
    }
    /** @trace $s */
    return (string) $s;
//  ^^^^^^^^^^^^^^^^^^^ Trace: Type of $s is string|null
}

/**
 * @param Missing $m
 */
function takes_missing($m = null): void
//       ^^^^^^^^^^^^^ UndefinedDocblockClass: Docblock type 'Missing' does not exist
//                     ^^^^^^^^^ UnusedParam: Parameter $m is never used
{
    func_get_args();
}

final class K
{
    /**
     * @param string|null $s
     * @param Missing $m
     */
    public function m($s = null, $m = null): string
//                  ^ UndefinedDocblockClass: Docblock type 'Missing' does not exist
//                               ^^^^^^^^^ UnusedParam: Parameter $m is never used
    {
        if (func_num_args() === 0) {
            return '';
        }
        /** @trace $s */
        return (string) $s;
//      ^^^^^^^^^^^^^^^^^^^ Trace: Type of $s is string|null
    }
}
