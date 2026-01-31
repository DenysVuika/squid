// Example JavaScript code with intentional issues for code review testing

// Issue: Using var instead of const/let
var globalCounter = 0;

// Issue: No input validation, uses eval (security risk)
function calculateExpression(expression) {
    return eval(expression);
}

// Issue: Not using strict equality
function checkValue(value) {
    if (value == 10) {
        return true;
    }
    return false;
}

// Issue: Callback pyramid (callback hell)
function getUserWithPosts(userId, callback) {
    fetchUser(userId, function(user) {
        fetchPosts(user.id, function(posts) {
            fetchComments(posts[0].id, function(comments) {
                callback({ user, posts, comments });
            });
        });
    });
}

// Issue: Mutating parameters
function addItem(array, item) {
    array.push(item);
    return array;
}

// Issue: Not using arrow functions appropriately
const numbers = [1, 2, 3, 4, 5];
const doubled = numbers.map(function(n) {
    return n * 2;
});

// Issue: No error handling for promises
function loadData(url) {
    return fetch(url)
        .then(response => response.json())
        .then(data => data);
}

// Issue: Using innerHTML with user content (XSS vulnerability)
function displayMessage(message) {
    document.getElementById('message').innerHTML = message;
}

// Issue: Memory leak - event listener never removed
function setupClickHandler() {
    const button = document.getElementById('btn');
    button.addEventListener('click', function() {
        console.log('Clicked');
    });
}

// Issue: Inefficient loops and operations
function processUsers(users) {
    var result = [];
    for (var i = 0; i < users.length; i++) {
        if (users[i].age > 18) {
            result.push(users[i]);
        }
    }
    var names = [];
    for (var j = 0; j < result.length; j++) {
        names.push(result[j].name.toUpperCase());
    }
    return names;
}

// Issue: No destructuring, repetitive code
function formatUser(user) {
    return {
        name: user.name,
        email: user.email,
        age: user.age,
        city: user.address.city,
        country: user.address.country
    };
}

// Issue: Magic numbers, no constants
function calculatePrice(quantity) {
    if (quantity > 100) {
        return quantity * 9.99 * 0.8;
    } else if (quantity > 50) {
        return quantity * 9.99 * 0.9;
    } else if (quantity > 10) {
        return quantity * 9.99 * 0.95;
    }
    return quantity * 9.99;
}

// Issue: Using for...in on arrays
function sumArray(arr) {
    var sum = 0;
    for (var index in arr) {
        sum += arr[index];
    }
    return sum;
}

// Issue: Not returning consistent types
function findUser(id) {
    const users = [{ id: 1, name: 'John' }, { id: 2, name: 'Jane' }];
    for (let user of users) {
        if (user.id == id) {
            return user;
        }
    }
    return false; // Should return null or undefined
}

// Issue: Modifying global state
let appState = { count: 0 };

function incrementCounter() {
    appState.count++;
}

// Issue: No null/undefined checks
function getUserName(user) {
    return user.name.toUpperCase();
}

// Issue: Unnecessary intermediate variables
function getFullName(firstName, lastName) {
    const first = firstName;
    const last = lastName;
    const fullName = first + ' ' + last;
    return fullName;
}

// Issue: Using setTimeout without cleanup, potential race condition
function delayedUpdate(value) {
    setTimeout(function() {
        appState.count = value;
    }, 1000);
}

// Helper functions (simulated)
function fetchUser(id, callback) { callback({ id, name: 'User' }); }
function fetchPosts(userId, callback) { callback([{ id: 1, userId }]); }
function fetchComments(postId, callback) { callback([{ id: 1, postId }]); }
