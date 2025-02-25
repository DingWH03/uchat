import React from 'react';
import { Link } from 'react-router-dom';

const Sidebar: React.FC = () => {
  return (
    <div className="sidebar">
      <h2>侧边栏</h2>
      <ul>
        <li><Link to="/home/friendlist">好友列表</Link></li>
        <li><Link to="/home/messagelist">消息列表</Link></li>
      </ul>
    </div>
  );
};

export default Sidebar;
