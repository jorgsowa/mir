===description===
@param-out on a class method: the out-type is written back to the caller's
variable after the method call.
===config===
suppress=UnusedVariable
===file===
<?php
class Parser {
    /**
     * @param-out list<string> $tokens
     */
    public function tokenize(string $input, mixed &$tokens): void {
        $tokens = explode(' ', $input);
    }
}

$p = new Parser();
$p->tokenize("hello world", $result);
/** @mir-check $result is list<string> */
$_ = $result;
===expect===
