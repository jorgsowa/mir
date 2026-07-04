===description===
interface_exists($var) narrows the variable from string to the more precise
interface-string (not the wider class-string) in the true branch, after a
negative early-exit, and leaves the false branch unchanged.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test_true_branch(string $iface): void {
    if (interface_exists($iface)) {
        /** @mir-check $iface is interface-string */
        $_ = $iface;
    }
}

function test_negative_early_exit(string $iface): void {
    if (!interface_exists($iface)) {
        return;
    }
    /** @mir-check $iface is interface-string */
    $_ = $iface;
}

function test_false_branch_stays_string(string $iface): void {
    if (interface_exists($iface)) {
        $_ = null;
    } else {
        /** @mir-check $iface is string */
        $_ = $iface;
    }
}

function test_already_interface_string(string $iface): void {
    /** @var interface-string $iface */
    if (interface_exists($iface)) {
        /** @mir-check $iface is interface-string */
        $_ = $iface;
    }
}
===expect===
