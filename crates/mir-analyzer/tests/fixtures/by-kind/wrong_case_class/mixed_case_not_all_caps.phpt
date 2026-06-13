===description===
Mixed case variants (not all-caps) are also detected.
===config===
suppress=UnusedVariable
===file===
<?php
class HttpClient {}
$c = new httpclient();
$d = new HttpCLIENT();
===expect===
WrongCaseClass@3:10-3:20: Class name 'httpclient' has incorrect casing; use 'HttpClient'
WrongCaseClass@4:10-4:20: Class name 'HttpCLIENT' has incorrect casing; use 'HttpClient'
