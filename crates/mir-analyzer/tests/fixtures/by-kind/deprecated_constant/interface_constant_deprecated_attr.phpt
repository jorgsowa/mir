===description===
FN: interface constants never checked the #[Deprecated] attribute
fallback, unlike class constants — only the @deprecated docblock tag
worked.
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
interface Flags {
    #[Deprecated]
    const OLD_FLAG = 1;
}

$v = Flags::OLD_FLAG;
===expect===
DeprecatedConstant@7:12-7:20: Constant Flags::OLD_FLAG is deprecated
