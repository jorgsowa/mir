===description===
Unused catch variables (e.g. `catch (Exception $e)`) must not produce UnusedVariable —
they are bound by the clause itself, not by explicit developer assignment.
The fix should hold even when the try/catch is nested inside an if-statement, which
triggers additional merge_branches calls that would otherwise drop the suppression.
===file===
<?php
function test(bool $flag): bool {
    if ($flag) {
        try {
            return true;
        } catch (\Exception $e) {
            return false;
        }
    }
    return false;
}

function testSimple(): string {
    $result = 'default';
    try {
        $result = 'ok';
    } catch (\RuntimeException $ex) {
        // intentionally empty — $ex not used
    }
    return $result;
}
===expect===
