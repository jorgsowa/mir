===description===
A passing `ctype_*()` check or a fully anchored `preg_match()` allowlist
proves which characters a value holds, so sinks those characters can't break
out of are safe. An unanchored or open-ended pattern proves nothing, and `$`
still allows a trailing newline.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedFunctionCall errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function guards(mysqli $db): void {
    $id = $_GET['id'];
    if (ctype_digit($id)) {
        echo $id;
        mysqli_query($db, 'SELECT * FROM t WHERE id = ' . $id);
    }
    $slug = $_GET['slug'];
    if (!preg_match('/^[a-z0-9-]+$/D', $slug)) {
        return;
    }
    echo $slug;
    mysqli_query($db, "SELECT * FROM t WHERE slug = '" . $slug . "'");
    header('X-Slug: ' . $slug);
}

function weakGuards(): void {
    $name = $_GET['name'];
    if (preg_match('/[a-z]+/', $name)) {
        echo $name;
//      ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    $tag = $_GET['tag'];
    if (preg_match('/^.+$/', $tag)) {
        echo $tag;
//      ^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
    $lang = $_GET['lang'];
    if (preg_match('/^[a-z]{2}$/', $lang)) {
        echo $lang;
        header('Content-Language: ' . $lang);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
    }
    $word = $_GET['word'];
    if (ctype_alpha($word)) {
        $word();
//      ^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
    }
    $other = $_GET['other'];
    if (ctype_digit($id = $_GET['id'])) {
        echo $other;
//      ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
