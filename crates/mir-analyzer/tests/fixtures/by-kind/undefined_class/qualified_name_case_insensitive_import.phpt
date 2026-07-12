===description===
qualified name resolves against use import case-insensitively
===config===
suppress=UnusedVariable
===file:Lib.php===
<?php
namespace MyApp\Deep;
class Service {}
===file:Main.php===
<?php
namespace Client;
use MyApp\Deep;
$x = new deep\Service();
===expect===
