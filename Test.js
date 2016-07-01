(function() {
  var Test = Calendar.createNS("Calendar.Test");
  Test.browser = false;
  Test.console = false;
  var report = function(name, result, answer, pass, err) {
    var body = Test.browser && Test.browser.document.getElementsByTagName(
      'body')[0] || false;
    if (body) {
      body.appendChild(window.document.createElement("hr"));
      var p = window.document.createElement("p");
      p.innerHTML = "测试项目：" + name + "<br/>";
      p.innerHTML += "输出结果：" + result + "<br/>";
      p.innerHTML += "参考结果：" + answer + "<br/>";
      p.innerHTML += "允许误差：" + err + "<br/>";
      p.innerHTML += "测试结果：" + (pass ? "<font color='green'>PASS</font>" :
        "<font color='red'>Failed</font>") + "<br/>";
      body.appendChild(p);
    }
    if (Test.console) {
      var console = Test.console;
      console.log("测试项目：" + name);
      console.log("输出结果：" + result);
      console.log("参考结果：" + answer);
      console.log("允许误差：" + err);
      console.log("测试结果：" + (pass ? "PASS" : "Failed"));
    };
  };
  Test.println = function(msg, tag) {
    var body = Test.browser && Test.browser.document.getElementsByTagName(
        'body')[0] ||
      false;
    if (body) {
      tag = tag || "p";
      var p = window.document.createElement(tag);
      p.innerHTML = msg;
      body.appendChild(p);
    }
    if (Test.console) {
      Test.console.log(msg);
    }
  }
  Test.checkNumber = function(a, b, err) {
    return Math.abs(a - b) < err;
  }
  Test.test = function(name, result, answer, err, err_str, check) {
    var pass = check && check() || Test.checkNumber(result, answer, err);
    report(name, result, answer, pass, err_str);
    return this;
  };
})();
