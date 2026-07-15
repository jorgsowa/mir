===description===
class_exists()/interface_exists()/trait_exists() narrow case-insensitively
(PHP calls are case-insensitive); enum_exists() narrows like class_exists().
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

function test_class_exists_mixed_case(string $cls): void {
    if (Class_Exists($cls)) {
        /** @mir-check $cls is class-string */
        $_ = $cls;
    }
}

function test_interface_exists_upper_case(string $iface): void {
    if (INTERFACE_EXISTS($iface)) {
        /** @mir-check $iface is interface-string */
        $_ = $iface;
    }
}

function test_trait_exists_mixed_case(string $tr): void {
    if (Trait_Exists($tr)) {
        /** @mir-check $tr is class-string */
        $_ = $tr;
    }
}

function test_enum_exists_true_branch(string $en): void {
    if (enum_exists($en)) {
        /** @mir-check $en is class-string */
        $_ = $en;
    }
}

function test_enum_exists_mixed_case(string $en): void {
    if (Enum_Exists($en)) {
        /** @mir-check $en is class-string */
        $_ = $en;
    }
}

function test_enum_exists_false_branch_stays_string(string $en): void {
    if (enum_exists($en)) {
        $_ = null;
    } else {
        /** @mir-check $en is string */
        $_ = $en;
    }
}
===expect===
WrongCaseFunction@4:8-4:20: Function name 'Class_Exists' has incorrect casing; use 'class_exists'
WrongCaseFunction@11:8-11:24: Function name 'INTERFACE_EXISTS' has incorrect casing; use 'interface_exists'
WrongCaseFunction@18:8-18:20: Function name 'Trait_Exists' has incorrect casing; use 'trait_exists'
WrongCaseFunction@32:8-32:19: Function name 'Enum_Exists' has incorrect casing; use 'enum_exists'
