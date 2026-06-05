===description===
Wrong case class name in return type hint is reported.
===file===
<?php
class Response {}
function build(): RESPONSE { return new Response(); }
===expect===
WrongCaseClass@3:19-3:27: Class name 'RESPONSE' has incorrect casing; use 'Response'
InvalidReturnType@3:30-3:52: Return type 'Response' is not compatible with declared 'RESPONSE'
