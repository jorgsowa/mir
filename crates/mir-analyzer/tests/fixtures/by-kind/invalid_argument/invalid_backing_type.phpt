===description===
Invalid backing type
===file===
<?php
enum Status: array {}

===expect===
ParseError@2:13-2:18: Parse error: Enum backing type must be int or string
