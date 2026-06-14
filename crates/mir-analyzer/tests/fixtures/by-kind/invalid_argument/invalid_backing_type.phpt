===description===
Invalid backing type
===file===
<?php
enum Status: array {}

===expect===
ParseError@2:14-2:19: Parse error: Enum backing type must be int or string
