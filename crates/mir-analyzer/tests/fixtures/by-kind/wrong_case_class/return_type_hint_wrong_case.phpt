===description===
Wrong case class name in return type hint is reported.
===file===
<?php
class Response {}
function build(): RESPONSE { return new Response(); }
//                ^^^^^^^^ WrongCaseClass: Class name 'RESPONSE' has incorrect casing; use 'Response'
