===description===
Mixed case variants (not all-caps) are also detected.
===file===
<?php
class HttpClient {}
new httpclient();
new HttpCLIENT();
===expect===
WrongCaseClass@3:5-3:15: Class name 'httpclient' has incorrect casing; use 'HttpClient'
WrongCaseClass@4:5-4:15: Class name 'HttpCLIENT' has incorrect casing; use 'HttpClient'
