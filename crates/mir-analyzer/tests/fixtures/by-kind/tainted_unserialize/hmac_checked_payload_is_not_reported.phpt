===description===
A payload that matched its `hash_hmac` under an untainted key in `hash_equals` is safe to unserialize.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$p = $_GET['p'];
if (hash_equals(hash_hmac('sha256', $p, 'k'), $_GET['s'])) { unserialize($p); }

$q = $_COOKIE['q'];
if (!hash_equals($_GET['s'], hash_hmac('sha256', $q, 'k'))) { exit; }
unserialize($q);
echo $q;
//<^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS

$r = $_GET['r'];
if (hash_equals(hash_hmac('sha256', $r, $_GET['k']), $_GET['s'])) { unserialize($r); }
//                                                                  ^^^^^^^^^^^^^^^ TaintedUnserialize: Tainted unserialize() payload — possible object injection
