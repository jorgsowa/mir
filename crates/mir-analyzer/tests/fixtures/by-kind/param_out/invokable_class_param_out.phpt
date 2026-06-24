===description===
Invokable class (__invoke with @param-out): calling via variable uses the
declared @param-out type, not the in-type (mixed), for the writeback.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
class Filler {
    /**
     * @param-out string $result
     */
    public function __invoke(mixed &$result): void {
        $result = "hello";
    }
}

$fn = new Filler();
$fn($out);
/** @mir-check $out is string */
$_ = $out;
===expect===
