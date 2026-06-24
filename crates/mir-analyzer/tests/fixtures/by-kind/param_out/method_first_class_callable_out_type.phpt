===description===
@param-out type is preserved when an instance method is captured as a
first-class callable. $result should be string (from @param-out string), not mixed.
===config===
suppress=UnusedVariable
===file===
<?php
class Formatter {
    /**
     * @param-out string $out
     */
    public function format(mixed &$out, string $prefix): void {
        $out = $prefix . '!';
    }
}

$f = new Formatter();
$fn = $f->format(...);
$fn($result, 'hello');
/** @mir-check $result is string */
echo $result;
===expect===
