// 创建根命名空间，保证不被覆盖
var Calendar = Calendar || {};

// 创建子命名空间的方法
Calendar.createNS = function(namespace) {
  var nsparts = namespace.split(".");
  var parent = Calendar;
  //去掉根命名空间
  if (nsparts[0] === "Calendar") {
    nsparts = nsparts.slice(1);
  }
  //循环创建嵌套的命名空间
  for (var i = 0; i < nsparts.length; i++) {
    var partname = nsparts[i];
    //命名空间是否已存在
    if (typeof parent[partname] === "undefined") {
      parent[partname] = {};
    }
    parent = parent[partname];
  }
  //返回最底层子空间
  return parent;
};
// 引用命名空间的方法
Calendar.importNS = function(namespace) {
  var nsparts = namespace.split(".");
  var parent = Calendar;
  //去掉根命名空间
  if (nsparts[0] === "Calendar") {
    nsparts = nsparts.slice(1);
  }
  //访问嵌套的命名空间
  for (var i = 0; i < nsparts.length; i++) {
    var partname = nsparts[i];
    //命名空间是否已存在
    if (typeof parent[partname] === "undefined") {
      //不存在， 报错
      throw new ReferenceError("NameSpace '" + namespace +
        "' does not exist.");
    }
    parent = parent[partname];
  }
  //返回最底层子空间
  return parent;
};
