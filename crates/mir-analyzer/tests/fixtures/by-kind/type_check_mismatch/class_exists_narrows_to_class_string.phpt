===description===
FP-A(b): class_exists($var) / interface_exists($var) / trait_exists($var) narrow the
variable from string to class-string in the true branch.  @mir-check assertions
verify the narrowed type inside the guard, after a negative early-exit, and that
the false branch is unchanged.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

function test_class_exists_true_branch(string $cls): void {
    if (class_exists($cls)) {
        /** @mir-check $cls is class-string */
        $_ = $cls;
    }
}

function test_interface_exists_true_branch(string $iface): void {
    if (interface_exists($iface)) {
        /** @mir-check $iface is class-string */
        $_ = $iface;
    }
}

function test_trait_exists_true_branch(string $tr): void {
    if (trait_exists($tr)) {
        /** @mir-check $tr is class-string */
        $_ = $tr;
    }
}

function test_negative_early_exit(string $cls): void {
    if (!class_exists($cls)) {
        return;
    }
    /** @mir-check $cls is class-string */
    $_ = $cls;
}

function test_negative_throw_guard(string $cls): void {
    if (!class_exists($cls)) {
        throw new \RuntimeException("Class $cls does not exist");
    }
    /** @mir-check $cls is class-string */
    $_ = $cls;
}

function test_false_branch_stays_string(string $cls): void {
    if (class_exists($cls)) {
        $_ = null;
    } else {
        // In the false branch class_exists returned false — $cls is still string
        /** @mir-check $cls is string */
        $_ = $cls;
    }
}

function test_already_class_string(string $cls): void {
    /** @var class-string $cls */
    if (class_exists($cls)) {
        // Already class-string — stays class-string
        /** @mir-check $cls is class-string */
        $_ = $cls;
    }
}
===expect===

