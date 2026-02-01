// Example TypeScript code with intentional issues for code review testing

interface User {
    name: string;
    age: number;
    email: string;
}

// Issue: Using 'any' type
export function processData(data: any) {
    return data.value;
}

// Issue: Not using proper async/await error handling
export async function fetchUserData(userId: string) {
    const response = await fetch(`/api/users/${userId}`);
    const data = await response.json();
    return data;
}

// Issue: Using 'var' instead of 'const'/'let'
export function calculateTotal(items: number[]) {
    var total = 0;
    for (var i = 0; i < items.length; i++) {
        total += items[i];
    }
    return total;
}

// Issue: Not properly typing function parameters and return
export function greetUser(user) {
    return "Hello, " + user.name + "!";
}

// Issue: Potential XSS vulnerability
export function renderUserContent(content: string) {
    document.getElementById('content')!.innerHTML = content;
}

// Issue: Memory leak - event listener not cleaned up
export class Counter {
    private count: number = 0;

    constructor() {
        window.addEventListener('click', () => {
            this.count++;
        });
    }

    getCount() {
        return this.count;
    }
}

// Issue: Inefficient array operations
export function filterAndMap(users: User[]) {
    const adults = users.filter(u => u.age >= 18);
    const names = adults.map(u => u.name);
    const upperNames = names.map(n => n.toUpperCase());
    return upperNames;
}

// Issue: Not using destructuring
export function getUserInfo(user: User) {
    return {
        name: user.name,
        email: user.email,
        age: user.age
    };
}

// Issue: Magic numbers and no input validation
export function calculateDiscount(price: number, quantity: number) {
    if (quantity > 10) {
        return price * 0.9;
    } else if (quantity > 5) {
        return price * 0.95;
    }
    return price;
}

// Issue: Using == instead of ===
export function compareValues(a: any, b: any) {
    return a == b;
}

// Issue: Not handling null/undefined properly
export function getNameLength(user: User | null): number {
    return user.name.length;
}

// Issue: Callback hell instead of async/await
export function loadUserData(userId: string, callback: Function) {
    fetch(`/api/users/${userId}`)
        .then(response => response.json())
        .then(user => {
            fetch(`/api/posts/${userId}`)
                .then(response => response.json())
                .then(posts => {
                    callback({ user, posts });
                });
        });
}

// Issue: Not following single responsibility principle
export class UserManager {
    users: User[] = [];

    addUser(user: User) {
        this.users.push(user);
    }

    saveToDatabase(user: User) {
        // database logic
        console.log('Saving to DB...');
    }

    sendEmailNotification(user: User) {
        // email logic
        console.log('Sending email...');
    }

    validateEmail(email: string) {
        return email.includes('@');
    }

    renderUserList() {
        // UI rendering logic
        return '<div>Users</div>';
    }
}
