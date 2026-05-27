===description===
does not report this when var-annotated in global scope (template/view pattern)
===file===
<?php
/** @var \yii\web\View $this */
$this->title = 'Processing...';
/** @mir-check $this is \yii\web\View */
$this->render('index');
echo $this->title;
===expect===
