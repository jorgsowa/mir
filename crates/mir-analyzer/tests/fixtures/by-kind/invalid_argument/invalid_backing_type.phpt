===description===
Invalid backing type
===file===
<?php
enum Status: array {}
//           ^^^^^ CompileError: Enum backing type must be int or string, array given [invalid-enum-backing-type]
