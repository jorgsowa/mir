===description===
collect_enum never called emit_docblock_issues, unlike class/trait/interface
— a malformed enum-level tag silently passed through.
===file===
<?php
interface Container {}

/**
 * @implements Container<int
 */
enum Status implements Container { case A; case B; }
===expect===
InvalidDocblock@4:0-4:0: Invalid docblock: @implements has unclosed generic type `Container<int`
