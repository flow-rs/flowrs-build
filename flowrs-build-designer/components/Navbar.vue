<template>
 <nav>
    <div class="nav-container">
      
      <ul class="nav-menu">
        <li><nuxt-link to="/">Home</nuxt-link></li>
        <li><nuxt-link to="/ProjectSelect">Project Select</nuxt-link></li>
        <li><nuxt-link to="/PackageSelect">Package Select</nuxt-link></li>
      </ul>
      <div class="logo">
         <div id="logo"><img src="../assets/images/flowers-logo.png" height="30PX"/></div>
      </div>
    </div>
  </nav>

</template>

<script>
export default {
  name: 'Navbar',
}
</script>

<style scoped>
nav {
  background-color: #ffffff;
  font-color: #000000;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.nav-container {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
}

.logo {
  font-size: 1.5rem;
  text-transform: uppercase;
}

.nav-menu {
  list-style: none;
  display: flex;
}

.nav-menu li {
  margin-right: 20px;
}

a {
  text-decoration: none;
  color: black;
  font-weight: bold;
  transition: color 0.3s;
}

a:hover {
  color: #000fff;
}
</style>